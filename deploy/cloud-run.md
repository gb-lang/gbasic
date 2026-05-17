# GBasic Compile Service on Cloud Run

The playground is static on GitHub Pages. The Run button calls this Cloud Run
service for `POST /compile` and telemetry.

## Current Deployment

- Google Cloud project: `gbasic-compile-sg3`
- Region: `us-central1`
- Artifact Registry repo: `gbasic`
- Cloud Run service: `gbasic-compile`
- Service URL: `https://gbasic-compile-vshks5pbha-uc.a.run.app`

## Build Image

```sh
IMAGE=us-central1-docker.pkg.dev/gbasic-compile-sg3/gbasic/compile-service:latest

gcloud builds submit \
  --project gbasic-compile-sg3 \
  --config deploy/cloudbuild.compile.yaml \
  --substitutions _IMAGE="$IMAGE" \
  .
```

## Deploy Service

```sh
gcloud run deploy gbasic-compile \
  --project gbasic-compile-sg3 \
  --region us-central1 \
  --image "$IMAGE" \
  --platform managed \
  --allow-unauthenticated \
  --port 8080 \
  --memory 1Gi \
  --cpu 1 \
  --concurrency 4 \
  --timeout 15s \
  --min-instances 0 \
  --max-instances 1 \
  --set-env-vars RUST_LOG=info,GBASIC_BIN=/usr/local/bin/gbasic
```

## Wire GitHub Pages

```sh
SERVICE_URL="$(gcloud run services describe gbasic-compile \
  --project gbasic-compile-sg3 \
  --region us-central1 \
  --format='value(status.url)')"

gh variable set GBASIC_COMPILE_URL --repo gb-lang/gbasic --body "$SERVICE_URL/compile"
gh variable set GBASIC_TELEMETRY_URL --repo gb-lang/gbasic --body "$SERVICE_URL/telemetry"
gh workflow run "Playground Pages" --repo gb-lang/gbasic --ref main
```

## Smoke Test

```sh
curl -fsS "$SERVICE_URL/compile" \
  -H 'content-type: application/json' \
  -d '{"source":"print(\"hi\")"}' \
  | python3 -c 'import json,sys; obj=json.load(sys.stdin); print(obj.get("errors") or "ok")'
```

## Cost Controls

- `min-instances=0` keeps idle cost near zero.
- `max-instances=1` caps surprise scale-out.
- The service rate-limits to 10 compiles/min/IP.
- A $10/month budget alert exists for project `gbasic-compile-sg3`.
