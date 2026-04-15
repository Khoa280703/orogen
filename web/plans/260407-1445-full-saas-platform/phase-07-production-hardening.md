# Phase 7: Production Hardening

## Overview
- Priority: Critical (before launch)
- Status: complete
- Completed: 2026-04-07

## 1. Email Service (Optional — deferred)
- Not needed for MVP: Google OAuth handles auth, no email verify needed
- Add later if needed for payment notifications
- Keep as future enhancement

## 2. Rate Limiting + Abuse Protection
- Per-IP rate limit: 60 req/min for auth endpoints (prevent brute force)
- Per-user burst limit: 10 concurrent requests max
- Signup spam: simple honeypot field or hCaptcha
- API abuse: per-key rate limit separate from plan quota
- Rust: tower-governor or custom middleware

## 4. Deployment
```yaml
# docker-compose.prod.yml
services:
  postgres:     # existing
  grok-api:     # Rust binary
    build: .
    env_file: .env
    depends_on: postgres
  web:          # Next.js
    build: ./web
    depends_on: grok-api
  nginx:        # reverse proxy + SSL
    image: nginx:alpine
    ports: ["80:80", "443:443"]
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./certs:/etc/ssl/certs
```
- SSL: Let's Encrypt certbot hoặc Cloudflare proxy
- Domain: point A record → server IP
- Dockerfile for Rust (multi-stage build)
- Dockerfile for Next.js

## 5. SEO
- `metadata` exports on all public pages (title, description, OG)
- `robots.txt`, `sitemap.xml` (next-sitemap)
- OG images for landing + docs
- Structured data (JSON-LD) for pricing

## 6. Error Pages
- `src/app/not-found.tsx` — custom 404
- `src/app/error.tsx` — custom 500
- Friendly messages + back-to-home link

## 7. Monitoring + Alerting
- Telegram bot: alert on account ban, proxy down, high error rate, payment received
- Rust: simple HTTP POST to Telegram Bot API
- Health check endpoint enhanced: include account/proxy status counts
- Cron: check health every 5 min, alert if degraded

## 8. Legal Pages
- `/terms` — Terms of Service
- `/privacy` — Privacy Policy
- Simple static MDX pages
- Footer links

## 9. Backup
- PostgreSQL: pg_dump daily cron → compressed file
- Retain 7 days rolling
- Script: `scripts/backup.sh`

## 10. Logging
- Rust: tracing-subscriber with JSON format for production
- Log rotation: logrotate or redirect to file with size limit
- Key fields: timestamp, request_id, user_id, api_key (masked), status, latency

## Files to Create
- `Dockerfile` (Rust multi-stage)
- `web/Dockerfile` (Next.js)
- `docker-compose.prod.yml`
- `nginx.conf`
- `scripts/backup.sh`
- `src/services/telegram.rs`
- `web/src/app/(public)/terms/page.mdx`
- `web/src/app/(public)/privacy/page.mdx`
- `web/src/app/not-found.tsx`
- `web/src/app/error.tsx`

## Implementation Steps
1. Rate limiting middleware
4. Dockerfiles (Rust + Next.js)
5. docker-compose.prod.yml + nginx
6. SEO metadata + sitemap
7. Error pages (404, 500)
8. Telegram alert bot
9. Legal pages (terms, privacy)
10. Backup script + cron
11. Production logging config

## Success Criteria
- Rate limiting blocks brute force attempts
- `docker compose -f docker-compose.prod.yml up` starts entire stack
- SSL terminates at nginx, serves on port 443
- Telegram alerts fire on account/proxy issues
- SEO: Google can index landing + docs
- Backup runs daily, verifiable restore
