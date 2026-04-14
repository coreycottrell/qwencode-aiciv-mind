# Data Pipeline Specification Template

---

## Pipeline: [Pipeline Name]

**Author:** Data Engineer
**Date:** [YYYY-MM-DD]
**Version:** 1.0
**Status:** Draft | In Review | Approved | Deployed

---

## 1. Overview

### 1.1 Purpose
[What business problem does this pipeline solve?]

### 1.2 Data Flow Summary
```
[Source A] ──┐
             ├──→ [Transform] ──→ [Destination]
[Source B] ──┘
```

### 1.3 Key Metrics
| Metric | Target |
|--------|--------|
| Freshness SLA | [e.g., Data available within 1 hour of source] |
| Volume | [e.g., ~10M records/day] |
| Uptime | [e.g., 99.9%] |

---

## 2. Data Sources

### 2.1 Source: [Source Name]

| Attribute | Value |
|-----------|-------|
| Type | Database / API / File / Stream |
| Connection | [Connection string / URL] |
| Format | JSON / CSV / Parquet / etc. |
| Frequency | Real-time / Hourly / Daily |
| Volume | [Records/day or GB/day] |
| Owner | [Team/Person responsible] |

**Schema:**
```sql
-- Key fields from source
field_name    TYPE        -- Description
field_name2   TYPE        -- Description
```

**Access:**
- Credentials: [Location in secrets manager]
- Permissions: [Required access level]

---

## 3. Transformations

### 3.1 Transformation Logic

| Step | Description | Input | Output |
|------|-------------|-------|--------|
| T1 | [Description] | [Source] | [Intermediate] |
| T2 | [Description] | [Intermediate] | [Final] |

### 3.2 Business Rules
1. [Rule 1: e.g., "Filter out records where status = 'deleted'"]
2. [Rule 2: e.g., "Convert timestamps to UTC"]
3. [Rule 3: e.g., "Deduplicate by id, keeping latest"]

### 3.3 Data Quality Rules
| Rule | Check | Action on Failure |
|------|-------|-------------------|
| DQ1 | Not null: `id` | Reject record |
| DQ2 | Valid range: `amount > 0` | Flag for review |
| DQ3 | Referential: `user_id` exists | Skip record |

---

## 4. Destination

### 4.1 Target: [Destination Name]

| Attribute | Value |
|-----------|-------|
| Type | Data Warehouse / Database / Lake |
| Platform | Snowflake / BigQuery / S3 / etc. |
| Database | [database_name] |
| Schema | [schema_name] |
| Table | [table_name] |

### 4.2 Target Schema
```sql
CREATE TABLE schema.table_name (
    id              VARCHAR(36)     PRIMARY KEY,
    field1          VARCHAR(255)    NOT NULL,
    field2          INTEGER,
    amount          DECIMAL(10,2),
    created_at      TIMESTAMP       NOT NULL,
    updated_at      TIMESTAMP       NOT NULL,
    _loaded_at      TIMESTAMP       DEFAULT CURRENT_TIMESTAMP
);
```

### 4.3 Load Strategy
- [ ] **Full Refresh** - Truncate and reload
- [ ] **Incremental** - Append new/changed records
- [ ] **Merge/Upsert** - Update existing, insert new
- [ ] **SCD Type 2** - Track history with versioning

**Incremental Key:** `updated_at`
**Unique Key:** `id`

---

## 5. Orchestration

### 5.1 Schedule
| Attribute | Value |
|-----------|-------|
| Frequency | Hourly / Daily / Real-time |
| Schedule | [Cron expression: `0 * * * *`] |
| Timezone | UTC |
| Start Time | [When pipeline should start] |

### 5.2 Dependencies
```
upstream_pipeline_1 ──┐
                      ├──→ [This Pipeline]
upstream_pipeline_2 ──┘
```

### 5.3 Orchestration Tool
- Platform: Airflow / Prefect / dbt Cloud
- DAG/Flow Name: `[dag_name]`
- Location: `dags/[filename].py`

---

## 6. Monitoring & Alerting

### 6.1 Health Checks
| Check | Threshold | Alert |
|-------|-----------|-------|
| Pipeline completion | Within SLA | Slack #data-alerts |
| Record count | ±20% of expected | Email |
| Data freshness | < 2 hours stale | PagerDuty |
| Error rate | < 1% | Slack |

### 6.2 Metrics Dashboard
- Location: [Link to Grafana/DataDog dashboard]

### 6.3 Logging
- Log location: [CloudWatch / GCS / etc.]
- Log retention: [30 days]

---

## 7. Error Handling

### 7.1 Retry Strategy
| Error Type | Retries | Backoff |
|------------|---------|---------|
| Network timeout | 3 | Exponential (1m, 5m, 15m) |
| Rate limit | 5 | Linear (60s) |
| Data error | 0 | Alert and skip |

### 7.2 Dead Letter Queue
- Location: [S3 bucket / table for failed records]
- Retention: [30 days]
- Review process: [How failures are investigated]

---

## 8. Security & Compliance

### 8.1 Data Classification
- [ ] Public
- [ ] Internal
- [ ] Confidential
- [ ] PII/Sensitive

### 8.2 PII Handling
| Field | PII Type | Treatment |
|-------|----------|-----------|
| email | Direct identifier | Hash/Mask |
| ip_address | Indirect identifier | Truncate |

### 8.3 Access Control
| Role | Access Level |
|------|--------------|
| Data Engineers | Read/Write |
| Analysts | Read only |
| BI Tools | Service account, read only |

---

## 9. Testing

### 9.1 Unit Tests
- [ ] Transformation logic tests
- [ ] Data quality rule tests

### 9.2 Integration Tests
- [ ] Source connectivity
- [ ] End-to-end data flow
- [ ] Destination write

### 9.3 Data Validation
```sql
-- Row count reconciliation
SELECT COUNT(*) FROM source WHERE date = '{{ ds }}'
SELECT COUNT(*) FROM destination WHERE _loaded_date = '{{ ds }}'
```

---

## 10. Runbook

### 10.1 Common Issues

**Issue: Pipeline timeout**
```
Cause: Large data volume or slow source
Fix:
1. Check source system health
2. Increase timeout parameter
3. Consider batching
```

**Issue: Data quality failures**
```
Cause: Source data issues
Fix:
1. Check DLQ for failed records
2. Investigate source changes
3. Update validation rules if needed
```

### 10.2 Manual Backfill
```bash
# Backfill specific date range
airflow dags backfill pipeline_name \
  --start-date 2026-01-01 \
  --end-date 2026-01-31
```

---

## 11. Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Name] | Initial release |
