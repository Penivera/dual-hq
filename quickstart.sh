#!/usr/bin/env bash
# ==============================================================================
# Dual HQ / Internship Application API - Quickstart Script
# ==============================================================================
set -eo pipefail

# ANSI color escape codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"

print_header() {
    echo -e "${BLUE}${BOLD}"
    echo "======================================================================"
    echo "           Dual HQ / Internship Application API Quickstart            "
    echo "======================================================================"
    echo -e "${NC}"
}

print_help() {
    print_header
    echo -e "Usage: ./quickstart.sh [COMMAND]"
    echo ""
    echo -e "Commands:"
    echo -e "  ${GREEN}up${NC} (default)    Build and start database & API services in background"
    echo -e "  ${GREEN}down | stop${NC}     Stop all running services and network"
    echo -e "  ${GREEN}restart${NC}         Restart all services"
    echo -e "  ${GREEN}logs${NC}            Follow runtime logs for all services"
    echo -e "  ${GREEN}status${NC}          Display status and health of running containers"
    echo -e "  ${GREEN}seed${NC}            Run initial admin database seed command"
    echo -e "  ${GREEN}help | -h${NC}       Show this help message"
    echo ""
}

check_prerequisites() {
    echo -e "${CYAN}--> Checking system dependencies...${NC}"
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}[ERROR] docker is not installed or not in PATH.${NC}"
        exit 1
    fi

    if ! docker compose version &> /dev/null; then
        echo -e "${RED}[ERROR] 'docker compose' is required but not available.${NC}"
        exit 1
    fi

    if ! command -v curl &> /dev/null; then
        echo -e "${RED}[ERROR] curl is required for health checks.${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Docker and curl prerequisites verified.${NC}"
}

prepare_environment() {
    echo -e "${CYAN}--> Setting up environment configuration...${NC}"
    if [ ! -f .env ]; then
        if [ -f .env.example ]; then
            echo -e "${YELLOW}Notice: .env not found. Copying .env.example to .env...${NC}"
            cp .env.example .env
        else
            echo -e "${YELLOW}Notice: Creating default .env...${NC}"
            cat << 'EOF' > .env
DATABASE_URL=${DATABASE_URL:-postgresql://postgres:admin@localhost:5432/internship_db}
JWT_SECRET=super-secret-jwt-key-minimum-32-chars-long-internship
JWT_EXPIRY_HOURS=24
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
ADMIN_EMAIL=admin@internship.local
ADMIN_PASSWORD=admin
ADMIN_NAME=Admin User
EOF
        fi
    fi

    # Ensure log directory exists
    mkdir -p logs

    # Ensure coolify network exists for local development
    if ! docker network inspect coolify >/dev/null 2>&1; then
        echo -e "${CYAN}--> Creating external 'coolify' Docker network...${NC}"
        docker network create coolify >/dev/null
    fi
    echo -e "${GREEN}✓ Environment and network ready.${NC}"
}

wait_for_service() {
    local port="${PORT:-8000}"
    local max_attempts=60
    local attempt=1
    local url="http://127.0.0.1:${port}/api/health"

    echo -e "${CYAN}--> Waiting for API server to become ready at ${url}...${NC}"
    while [ $attempt -le $max_attempts ]; do
        if curl -s -f "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ API server is healthy and responding! (Attempt ${attempt}/${max_attempts})${NC}"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done

    echo -e "${RED}[ERROR] Server did not become healthy within ${max_attempts} seconds.${NC}"
    echo -e "${YELLOW}Inspecting recent logs:${NC}"
    docker compose logs --tail=40
    exit 1
}

verify_endpoints() {
    local port="${PORT:-8000}"
    echo -e "${CYAN}--> Verifying endpoints...${NC}"

    # Verify Health
    local health_resp
    health_resp=$(curl -s "http://127.0.0.1:${port}/api/health" || true)
    echo -e "  - Health API:       ${GREEN}OK${NC} (${health_resp})"

    # Verify Docs
    local docs_code
    docs_code=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${port}/docs/" || true)
    echo -e "  - Swagger UI:       ${GREEN}HTTP ${docs_code}${NC}"

    # Verify Admin UI
    local admin_code
    admin_code=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${port}/admin/" || true)
    echo -e "  - SeaORM Pro Admin: ${GREEN}HTTP ${admin_code}${NC}"
}

show_summary() {
    local port="${PORT:-8000}"
    echo ""
    echo -e "${GREEN}${BOLD}======================================================================${NC}"
    echo -e "${GREEN}${BOLD}               🚀  All Services Running Successfully                 ${NC}"
    echo -e "${GREEN}${BOLD}======================================================================${NC}"
    echo ""
    echo -e "  ${BOLD}API Base URL:${NC}       http://localhost:${port}"
    echo -e "  ${BOLD}Swagger Docs:${NC}       http://localhost:${port}/docs"
    echo -e "  ${BOLD}Admin Dashboard:${NC}    http://localhost:${port}/admin"
    echo -e "  ${BOLD}Health Check:${NC}       http://localhost:${port}/api/health"
    echo ""
    echo -e "  ${BOLD}Initial Admin Account:${NC}"
    echo -e "    Email:    ${CYAN}admin@internship.local${NC}"
    echo -e "    Password: ${CYAN}admin${NC}"
    echo ""
    echo -e "  ${BOLD}Helpful Commands:${NC}"
    echo -e "    Follow logs:      ${YELLOW}./quickstart.sh logs${NC}"
    echo -e "    Check status:     ${YELLOW}./quickstart.sh status${NC}"
    echo -e "    Stop stack:       ${YELLOW}./quickstart.sh down${NC}"
    echo -e "======================================================================"
    echo ""
}

# Main command router
COMMAND="${1:-up}"

case "$COMMAND" in
    "up"|"start")
        print_header
        check_prerequisites
        prepare_environment
        echo -e "${CYAN}--> Building and starting containers via docker compose...${NC}"
        docker compose up -d --build
        wait_for_service
        verify_endpoints
        show_summary
        ;;
    "down"|"stop")
        print_header
        echo -e "${YELLOW}--> Stopping and removing containers...${NC}"
        docker compose down
        echo -e "${GREEN}✓ All services stopped.${NC}"
        ;;
    "restart")
        print_header
        echo -e "${YELLOW}--> Restarting containers...${NC}"
        docker compose restart
        wait_for_service
        verify_endpoints
        show_summary
        ;;
    "logs")
        docker compose logs -f
        ;;
    "status"|"ps")
        docker compose ps
        ;;
    "seed")
        print_header
        echo -e "${CYAN}--> Running admin seeder inside dual_hq_web container...${NC}"
        docker compose exec web /usr/local/bin/seed
        ;;
    "help"|"-h"|"--help")
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: ${COMMAND}${NC}"
        print_help
        exit 1
        ;;
esac
