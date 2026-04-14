# API Specification Template

---

## API: [API/Endpoint Name]

**Author:** [Agent Name]
**Date:** [YYYY-MM-DD]
**Version:** 1.0
**Status:** Draft | In Review | Approved | Implemented

---

## 1. Overview

### 1.1 Purpose
[What does this API do and why?]

### 1.2 Base URL
```
Development: https://dev.purebrain.ai/api
Staging: https://staging.purebrain.ai/api
Production: https://purebrain.ai/api
```

---

## 2. Authentication

### 2.1 Method
- [ ] API Key (Header: `X-API-Key`)
- [ ] Bearer Token (Header: `Authorization: Bearer <token>`)
- [ ] Session Cookie
- [ ] No Authentication (Public)

### 2.2 Authorization
[Who can access this endpoint? Roles/permissions required]

---

## 3. Endpoints

### 3.1 [Endpoint Name]

#### `[METHOD] /api/v1/[resource]`

**Description:** [What this endpoint does]

**Authentication:** Required / Not Required

**Request:**

| Parameter | Location | Type | Required | Description |
|-----------|----------|------|----------|-------------|
| `id` | path | string | Yes | Resource ID |
| `filter` | query | string | No | Filter criteria |
| `limit` | query | integer | No | Max results (default: 20) |

**Request Body:**
```json
{
  "field1": "string",
  "field2": 123,
  "nested": {
    "subfield": "value"
  }
}
```

**Response:**

**Success (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "abc123",
    "field1": "value",
    "createdAt": "2026-02-04T12:00:00Z"
  }
}
```

**Error Responses:**

| Status | Code | Description |
|--------|------|-------------|
| 400 | `INVALID_REQUEST` | Request validation failed |
| 401 | `UNAUTHORIZED` | Authentication required |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 404 | `NOT_FOUND` | Resource not found |
| 429 | `RATE_LIMITED` | Too many requests |
| 500 | `INTERNAL_ERROR` | Server error |

**Error Response Format:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Human-readable error message",
    "details": {
      "field": "Specific field error"
    }
  }
}
```

**Example:**
```bash
curl -X POST https://purebrain.ai/api/v1/resource \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"field1": "value"}'
```

---

### 3.2 [Next Endpoint]

#### `[METHOD] /api/v1/[resource]`

[Repeat structure above]

---

## 4. Data Models

### 4.1 [Model Name]

```typescript
interface ModelName {
  id: string;           // Unique identifier
  field1: string;       // Description
  field2: number;       // Description
  status: 'active' | 'inactive';
  createdAt: Date;
  updatedAt: Date;
}
```

---

## 5. Rate Limiting

| Tier | Requests/Minute | Requests/Day |
|------|-----------------|--------------|
| Free | 60 | 1,000 |
| Pro | 300 | 10,000 |
| Enterprise | Unlimited | Unlimited |

**Rate Limit Headers:**
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1707048000
```

---

## 6. Pagination

**Request:**
```
GET /api/v1/resources?page=1&limit=20
```

**Response:**
```json
{
  "success": true,
  "data": [...],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "totalPages": 5,
    "hasNext": true,
    "hasPrev": false
  }
}
```

---

## 7. Webhooks (if applicable)

### 7.1 Events

| Event | Description | Payload |
|-------|-------------|---------|
| `resource.created` | New resource created | [Resource object] |
| `resource.updated` | Resource modified | [Resource object] |
| `resource.deleted` | Resource removed | `{ id: string }` |

### 7.2 Webhook Payload
```json
{
  "event": "resource.created",
  "timestamp": "2026-02-04T12:00:00Z",
  "data": { }
}
```

---

## 8. Versioning

- Current version: `v1`
- Version in URL: `/api/v1/...`
- Deprecation policy: [X months notice]

---

## 9. Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | [Date] | Initial release |
