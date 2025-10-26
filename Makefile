COMPOSE ?= docker compose

.PHONY: up down build logs ps clean prune dev

# Bring up the entire stack (frontend, backend, database)
up:
	$(COMPOSE) up --build

# Bring up the stack with IA mock mode enabled
dev:
	IA_MOCK_MODE=true $(COMPOSE) up --build

# Stop services and remove containers (keeps volumes)
down:
	$(COMPOSE) down

# Build images without starting containers
build:
	$(COMPOSE) build

# Tail service logs
logs:
	$(COMPOSE) logs -f

# Show service status
ps:
	$(COMPOSE) ps

# Remove containers, networks, and persistent volumes
clean:
	$(COMPOSE) down -v --remove-orphans

# Stop services and prune Docker artifacts (containers, images, volumes, networks)
prune:
	$(COMPOSE) down -v --remove-orphans
	docker system prune -af --volumes
