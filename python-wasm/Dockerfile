# Python WASM Service Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install required packages
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy Python source code
COPY . .

# Create non-root user
RUN adduser --disabled-password --gecos '\' appuser && \
    chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 8001

# Run the Python service
CMD ["python", "service.py"]