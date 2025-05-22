FROM openjdk:21-slim
COPY ../execute/java_executor.sh /runner.sh
RUN chmod +x /runner.sh
ENTRYPOINT ["/runner.sh"]
