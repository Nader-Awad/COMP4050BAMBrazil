COMPOSE ?= docker compose

.PHONY: up down build logs ps clean prune dev up-ia down-ia build-ia logs-ia

# Bring up the entire stack (frontend, backend, database)
up:
	$(COMPOSE) up --build

# Bring up the API + database only with IA mock mode enabled for local UI dev
dev:
	IA_MOCK_MODE=true $(COMPOSE) up --build api db

# Bring up the IA stack in addition to the core services
up-ia:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.ia.yml up --build

# Stop services and remove containers (keeps volumes)
down:
	$(COMPOSE) down

# Stop the IA-extended stack
down-ia:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.ia.yml down

# Build images without starting containers
build:
	$(COMPOSE) build

# Build images including the IA service
build-ia:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.ia.yml build

# Tail service logs
logs:
	$(COMPOSE) logs -f

# Tail logs including the IA service
logs-ia:
	$(COMPOSE) -f docker-compose.yml -f docker-compose.ia.yml logs -f

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
