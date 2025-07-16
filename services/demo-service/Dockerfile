FROM alpine:latest

# Install basic utilities
RUN apk --no-cache add ca-certificates curl

WORKDIR /app

# Copy application files
COPY . .

# Make any scripts executable
RUN find . -name "*.sh" -exec chmod +x {} \;

EXPOSE 8080

# Default command - override in your specific implementation
CMD ["echo", "Please customize this Dockerfile for your specific application"]
