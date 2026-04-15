# Brainstorm: Grok API Service MVP

## Problem
Tận dụng Grok Heavy accounts ($1/acc) + Rust backend (wreq + SOCKS5 proxy) để bán API compatible OpenAI format.

## Key Findings
- wreq + US SOCKS5 proxy bypass CF thành công
- Heavy accounts có rate limit cao, chi phí thấp ($1 vs $25 retail)
- Thị trường unofficial Grok API: $1-3/1M tokens
- Competition: nhiều nhưng barrier thấp, ai cũng clone được

## Cost Structure
| Item | Cost/month |
|---|---|
| 100 Heavy accounts | ~$100 |
| 10-20 US proxy | ~$100-200 |
| VPS | ~$10-20 |
| **Total** | **~$200-320** |

## Revenue Estimate
- 100 accounts × 100-200 msg/day = 10-20K msg/day
- ~10-20M tokens/day → $10-60/day = $300-1,800/month
- **Margin: $100-1,500/month**

## Risks
- Account ban waves (🔴 high)
- CF detection upgrade (🟡 medium)  
- Price competition race to bottom (🟡 medium)
- TOS violation, no legal protection (🔴 high)

## Agreed Solution: MVP
1. Proxy pool — round-robin US proxies, bind per-account
2. Account pool — rotate on 429, pause on ban, health tracking
3. API server — /v1/chat/completions (SSE), /v1/models
4. API key auth — Bearer token from config
5. Request logging — count per account/key

**NOT in MVP:** billing, dashboard, auto-signup, payment.

## Architecture
```
User → Bearer Key → Axum → Account Pool → Proxy Pool → wreq → SOCKS5 → grok.com → SSE back
```

## Next Steps
- Create detailed implementation plan
- Implement in phases: proxy pool → account pool → API server → auth
