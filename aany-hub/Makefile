.PHONY: help setup run test clean proto

help:
	@echo "Available commands:"
	@echo "  make setup   - Set up development environment"
	@echo "  make run     - Run the server"
	@echo "  make test    - Run tests"
	@echo "  make proto   - Compile protobuf files"
	@echo "  make clean   - Clean generated files"

setup:
	@echo "Setting up development environment..."
	@./setup_dev.sh

run:
	@echo "Starting aany-hub server..."
	@./run_server.sh

test:
	@echo "Running tests..."
	@. .venv/bin/activate && pytest

proto:
	@echo "Compiling protobuf files..."
	@. .venv/bin/activate && python compile_proto.py

clean:
	@echo "Cleaning generated files..."
	@find . -type f -name "*_pb2.py" -delete
	@find . -type f -name "*_pb2_grpc.py" -delete
	@find . -type d -name "__pycache__" -exec rm -rf {} +
	@find . -type f -name "*.pyc" -delete
	@rm -rf logs/ data/

install-deps:
	@echo "Installing dependencies..."
	@. .venv/bin/activate && pip install -r requirements.txt

format:
	@echo "Formatting code..."
	@. .venv/bin/activate && black src/

lint:
	@echo "Linting code..."
	@. .venv/bin/activate && flake8 src/