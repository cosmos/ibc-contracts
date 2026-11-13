# Docker Monitoring with Grafana Alloy

A local observability stack (Grafana, Loki, Tempo, Prometheus and Grafana Alloy) for e2e runs. Based on the Grafana [alloy-scenarios](https://github.com/grafana/alloy-scenarios) docker-monitoring example. See `docs/tracing.md` for how to point the e2e proof API at it.
## Prerequisites
- Docker
- Docker Compose

## Running the Demo

### Step 1: Deploy the monitoring stack
```bash
cd scripts/local-grafana-stack
docker compose up -d
```

`docker-compose-linux.yml` is a variant for Linux hosts (no Tempo, Grafana on port 3000): `docker compose -f docker-compose-linux.yml up -d`. Tracing does not work with this variant: `config.alloy` still exports traces to `tempo:4317`, and nothing is listening there, so traces are dropped. Use the default `docker-compose.yml` if you need traces.

### Step 2: Access Grafana Alloy UI
Open your browser and go to `http://localhost:12345`. 

### Step 3: Access Grafana UI
Open your browser and go to `http://localhost:3002` (`http://localhost:3000` with `docker-compose-linux.yml`).
