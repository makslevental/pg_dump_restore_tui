use cursive::theme::{ColorStyle, Effect};
use cursive::view::View;
use cursive::Printer;
use std::cmp;
use std::thread;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::utils::Counter;
use cursive::views::{Dialog, OnEventView, ProgressBar, SelectView, TextView};
use cursive::Cursive;

pub struct Spinner {
    value: Counter,
}


impl Spinner {
    pub fn new() -> Self {
        Spinner {
            value: Counter::new(0),
        }
    }

    pub fn with_value(mut self, value: Counter) -> Self {
        self.value = value;
        self
    }

    pub fn start<F: FnOnce(Counter) + Send + 'static>(&mut self, f: F) {
        let counter: Counter = self.value.clone();

        thread::spawn(move || {
            f(counter);
        });
    }

    pub fn with_task<F: FnOnce(Counter) + Send + 'static>(mut self, task: F) -> Self {
        self.start(task);
        self
    }


    pub fn set_value(&mut self, value: usize) {
        self.value.set(value);
    }
}

impl View for Spinner {
    fn draw(&self, printer: &Printer) {
        let value = self.value.get();

        let offset = HAlign::Center.get_offset(1, printer.size.x);
        let spinner = vec![
            "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
        ];

        let spinner = vec![
            "🌑 ",
            "🌒 ",
            "🌓 ",
            "🌔 ",
            "🌕 ",
            "🌖 ",
            "🌗 ",
            "🌘 "];

        printer.print((offset, 0), spinner[value % spinner.len()]);
    }
}
