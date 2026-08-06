---
name: explicit-configuration
user-invocable: false
description: Prefer explicit configuration over framework defaults to prevent environment-dependent failures
allowed-tools: []
---

# Explicit Configuration

## Name
han-core:explicit-configuration

## Description

When configuring services, APIs, or framework features, explicitly set all parameters rather than relying on defaults. Defaults vary across versions, environments, and frameworks.

## Principle

What works with defaults in development may fail in production. Explicit configuration is documentation that runs.

## Examples

**Database connections:**
- Set pool size, timeout, retry policy explicitly
- Don't rely on default connection string parsing

**API calls:**
- Set timeout, retry count, headers explicitly
- Don't assume default content-type

**Framework config:**
- Set port, environment, logging level explicitly
- Don't rely on framework auto-detection

**Security settings:**
- Never rely on default CORS, auth, or session config
- Always configure explicitly, even if the default is "secure"

## Anti-Pattern

```
createServer()  // relies on default port, middleware, error handling
```

## Pattern

```
createServer({
  port: config.PORT,
  timeout: 30000,
  cors: { origin: config.ALLOWED_ORIGINS }
})
```

## When to Apply

- Any time you're initializing a service, client, or framework
- Any time you're configuring infrastructure (Docker, CI, cloud services)
- Any time the behavior difference between dev and production matters
