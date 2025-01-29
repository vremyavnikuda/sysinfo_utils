# Установка имени теста из первого аргумента командной строки
TEST_NAME := $(word 2, $(MAKECMDGOALS))

# Цель для запуска тестов
.PHONY: test
test:
	@if [ "$(TEST_NAME)" = "all" ]; then \
		RUST_BACKTRACE=full cargo test --release; \
	elif [ "$(TEST_NAME)" = "" ]; then \
		echo "Error: Please specify the test name or use 'all' to run all tests. Usage: make test <test_name|all>"; \
		exit 1; \
	else \
		RUST_BACKTRACE=full cargo test --release $(TEST_NAME); \
	fi

# Убираем обработку аргументов как целей
%:
	@:
