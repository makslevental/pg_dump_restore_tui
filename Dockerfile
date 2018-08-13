FROM postgres:9.6

RUN apt update && apt install -y python3 postgresql-plpython-9.6