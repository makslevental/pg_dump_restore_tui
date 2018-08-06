use std::cmp;
use std::thread;
use cursive::theme::{ColorStyle, Effect};
use cursive::view::View;
use cursive::Printer;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView, ProgressBar};
use cursive::Cursive;
use cursive::utils::Counter;

pub struct Spinner {
    min: usize,
    max: usize,
    value: Counter,
    // TODO: use a Promise instead?
//    label_maker: Box<Fn(usize, (usize, usize)) -> String>,
}

//new_default!(ProgressBar);

impl Spinner {
    /// Creates a new progress bar.
    ///
    /// Default values:
    ///
    /// * `min`: 0
    /// * `max`: 100
    /// * `value`: 0
    pub fn new() -> Self {
        Spinner {
            min: 0,
            max: 100,
            value: Counter::new(0),
//            label_maker: Box::new(make_percentage),
        }
    }

    /// Sets the value to follow.
    ///
    /// Use this to manually control the progress to display
    /// by directly modifying the value pointed to by `value`.
    pub fn with_value(mut self, value: Counter) -> Self {
        self.value = value;
        self
    }

    /// Starts a function in a separate thread, and monitor the progress.
    ///
    /// `f` will be given a `Counter` to increment the bar's progress.
    ///
    /// This does not reset the value, so it can be called several times
    /// to advance the progress in multiple sessions.
    pub fn start<F: FnOnce(Counter) + Send + 'static>(&mut self, f: F) {
        let counter: Counter = self.value.clone();

        thread::spawn(move || {
            f(counter);
        });
    }

    /// Starts a function in a separate thread, and monitor the progress.
    ///
    /// Chainable variant.
    pub fn with_task<F: FnOnce(Counter) + Send + 'static>(
        mut self, task: F,
    ) -> Self {
        self.start(task);
        self
    }

    /// Sets the label generator.
    ///
    /// The given function will be called with `(value, (min, max))`.
    /// Its output will be used as the label to print inside the progress bar.
    ///
    /// The default one shows a percentage progress:
    ///
    /// ```
    /// fn make_progress(value: usize, (min, max): (usize, usize)) -> String {
    ///     let percent = 101 * (value - min) / (1 + max - min);
    ///     format!("{} %", percent)
    /// }
    /// ```
//    pub fn with_label<F: Fn(usize, (usize, usize)) -> String + 'static>(
//        mut self, label_maker: F,
//    ) -> Self {
//        self.label_maker = Box::new(label_maker);
//        self
//    }

    /// Sets the minimum value.
    ///
    /// When `value` equals `min`, the bar is at the minimum level.
    ///
    /// If `self.min > max`, `self.min` is set to `max`.
    pub fn min(mut self, min: usize) -> Self {
        self.min = min;
        self.max = cmp::max(self.max, self.min);

        self
    }

    /// Sets the maximum value.
    ///
    /// When `value` equals `max`, the bar is at the maximum level.
    ///
    /// If `min > self.max`, `self.max` is set to `min`.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self.min = cmp::min(self.min, self.max);

        self
    }

    /// Sets the `min` and `max` range for the value.
    ///
    /// If `min > max`, swap the two values.
    pub fn range(self, min: usize, max: usize) -> Self {
        if min > max {
            self.min(max).max(min)
        } else {
            self.min(min).max(max)
        }
    }

    /// Sets the current value.
    ///
    /// Value is clamped between `min` and `max`.
    pub fn set_value(&mut self, value: usize) {
        self.value.set(value);
    }
}

fn sub_block(extra: usize) -> &'static str {
    match extra {
        0 => " ",
        1 => "▏",
        2 => "▎",
        3 => "▍",
        4 => "▌",
        5 => "▋",
        6 => "▊",
        7 => "▉",
        _ => "█",
    }
}

impl View for Spinner {
    fn draw(&self, printer: &Printer) {
        // Now, the bar itself...
//        let available = printer.size.x;

//        let value = self.value.get();

        // If we're under the minimum, don't draw anything.
        // If we're over the maximum, we'll try to draw more, but the printer
        // will crop us anyway, so it's not a big deal.
//        let (length, extra) = if value < self.min {
//            (0, 0)
//        } else {
//            ratio(value - self.min, self.max - self.min, available)
//        };
//
//        let label = (self.label_maker)(value, (self.min, self.max));
//        let offset = HAlign::Center.get_offset(label.len(), printer.size.x);

        let vale = 100;
        let length = 100;
        let offset = 100;
        let extra= 100;
        let label= "bob";

        printer.with_color(ColorStyle::highlight(), |printer| {
            // Draw the right half of the label in reverse
            printer.with_effect(Effect::Reverse, |printer| {
                printer.print((length, 0), sub_block(extra));
                printer.print((offset, 0), &label);
            });
            let printer = &printer.cropped((length, 1));
            printer.print_hline((0, 0), length, " ");

            // Draw the left part in highlight (it may be cropped)
            printer.print((offset, 0), &label);
        });
    }
}