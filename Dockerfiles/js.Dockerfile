FROM node:20-slim
COPY execute/js_executor.sh /runner.sh
RUN chmod +x /runner.sh
ENTRYPOINT ["/runner.sh"]
