FROM gcc:13
COPY ../execute/cpp_executor.sh /runner.sh
RUN chmod +x /runner.sh
ENTRYPOINT ["/runner.sh"]
