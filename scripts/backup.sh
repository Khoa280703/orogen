#!/bin/bash

# PostgreSQL Backup Script
# Run this daily via cron: 0 2 * * * /app/scripts/backup.sh

set -e

# Configuration
BACKUP_DIR="/backups/postgres"
RETENTION_DAYS=7
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="${POSTGRES_DB:-duanai}"
DB_USER="${POSTGRES_USER:-duanai}"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Export password for pg_dump
export PGPASSWORD="${POSTGRES_PASSWORD}"

# Create backup filename
BACKUP_FILE="${BACKUP_DIR}/${DB_NAME}_${DATE}.sql.gz"

echo "Starting backup at $(date)"

# Perform backup
pg_dump -h postgres -U "$DB_USER" "$DB_NAME" | gzip > "$BACKUP_FILE"

# Verify backup was created
if [ -f "$BACKUP_FILE" ]; then
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    echo "Backup completed successfully: $BACKUP_FILE ($BACKUP_SIZE)"
else
    echo "Backup failed!"
    exit 1
fi

# Clean up old backups
echo "Cleaning up backups older than $RETENTION_DAYS days"
find "$BACKUP_DIR" -name "*.sql.gz" -type f -mtime +$RETENTION_DAYS -delete

echo "Backup script completed at $(date)"
