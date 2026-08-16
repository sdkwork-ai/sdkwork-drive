import { Database } from "lucide-react";
import { useTranslation } from "react-i18next";

import type { CloudDriveStorageSummary } from "../services/CloudDriveService";

interface CloudDriveHeaderStatsProps {
  summary: CloudDriveStorageSummary | null;
}

const GIBIBYTE = 1024 * 1024 * 1024;

function formatGigabytes(bytes: number): string {
  return (bytes / GIBIBYTE).toFixed(1);
}

export function CloudDriveHeaderStats({ summary }: CloudDriveHeaderStatsProps) {
  const { t } = useTranslation("drive");
  const usedBytes = summary?.usedBytes ?? 0;
  const totalBytes = summary?.totalBytes;
  const usagePercent = totalBytes && totalBytes > 0
    ? Math.min(100, (usedBytes / totalBytes) * 100)
    : 0;
  const availableBytes = totalBytes === undefined
    ? undefined
    : Math.max(0, totalBytes - usedBytes);

  return (
    <div className="bg-primary-blue px-6 pt-4 pb-12 text-white">
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Database className="w-5 h-5 opacity-90" />
          <span className="font-medium text-[16px]">{t("storage_space")}</span>
        </div>
        <div className="text-[14px] opacity-80 font-mono">
          {totalBytes === undefined
            ? `${formatGigabytes(usedBytes)} GB`
            : `${formatGigabytes(usedBytes)} GB / ${formatGigabytes(totalBytes)} GB`}
        </div>
      </div>
      <div className="w-full bg-white/20 rounded-full h-2 overflow-hidden mb-2">
        <div
          className="bg-white h-full rounded-full"
          style={{ width: `${usagePercent}%` }}
        />
      </div>
      <div className="flex justify-between text-[12px] opacity-70">
        <span>{t("storage_used", { percent: usagePercent.toFixed(1) })}</span>
        <span>
          {availableBytes === undefined
            ? t("storage_available_unknown")
            : t("storage_available", { available: formatGigabytes(availableBytes) })}
        </span>
      </div>
    </div>
  );
}
