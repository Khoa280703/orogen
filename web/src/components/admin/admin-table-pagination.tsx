'use client';

import { Button } from '@/components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface AdminTablePaginationProps {
  page: number;
  pageSize: number;
  visibleCount: number;
  totalCount?: number;
  hasNextPage?: boolean;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  pageSizeOptions?: number[];
}

export function AdminTablePagination({
  page,
  pageSize,
  visibleCount,
  totalCount,
  hasNextPage,
  onPageChange,
  onPageSizeChange,
  pageSizeOptions = [10, 20, 50, 100],
}: AdminTablePaginationProps) {
  const start = visibleCount === 0 ? 0 : (page - 1) * pageSize + 1;
  const end = visibleCount === 0 ? 0 : start + visibleCount - 1;
  const canGoPrevious = page > 1;
  const canGoNext = typeof totalCount === 'number' ? end < totalCount : Boolean(hasNextPage);

  return (
    <div className="flex flex-col gap-3 border-t border-slate-200 pt-4 text-sm text-slate-500 lg:flex-row lg:items-center lg:justify-between">
      <div className="flex flex-wrap items-center gap-3">
        <span>
          {typeof totalCount === 'number'
            ? `Showing ${start}-${end} of ${totalCount}`
            : `Showing ${start}-${end}`}
        </span>
        <div className="flex items-center gap-2">
          <span>Rows</span>
          <Select value={String(pageSize)} onValueChange={(value) => onPageSizeChange(Number(value))}>
            <SelectTrigger className="w-20">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {pageSizeOptions.map((option) => (
                <SelectItem key={option} value={String(option)}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm" disabled={!canGoPrevious} onClick={() => onPageChange(page - 1)}>
          Previous
        </Button>
        <span className="min-w-16 text-center">Page {page}</span>
        <Button variant="outline" size="sm" disabled={!canGoNext} onClick={() => onPageChange(page + 1)}>
          Next
        </Button>
      </div>
    </div>
  );
}
