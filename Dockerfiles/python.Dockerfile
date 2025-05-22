FROM python:3.11-slim
COPY ../execute/python_executor.sh /runner.sh
RUN chmod +x /runner.sh
ENTRYPOINT ["/runner.sh"]
