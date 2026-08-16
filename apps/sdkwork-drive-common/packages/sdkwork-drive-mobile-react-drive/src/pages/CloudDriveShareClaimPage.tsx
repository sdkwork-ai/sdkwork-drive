import { useEffect, useState } from "react";
import { HardDrive } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router";

import { PageLayout } from "@sdkwork/ui-mobile-react";

import { CloudDriveService } from "../services/CloudDriveService";

type ClaimState = "claiming" | "claimed" | "failed";

export function CloudDriveShareClaimPage() {
  const { t } = useTranslation("drive");
  const navigate = useNavigate();
  const { token = "" } = useParams<{ token: string }>();
  const [state, setState] = useState<ClaimState>("claiming");

  useEffect(() => {
    if (!token) {
      setState("failed");
      return undefined;
    }
    let cancelled = false;
    void CloudDriveService.claimShareLink(token)
      .then(() => {
        if (!cancelled) {
          setState("claimed");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setState("failed");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [token]);

  return (
    <PageLayout title={t("share_claim_title")}>
      <div className="flex h-full flex-col items-center justify-center gap-4 bg-[#f5f6f8] px-6 text-center dark:bg-[#1a1b1c]">
        <HardDrive className="h-12 w-12 text-primary-blue" />
        <p className="text-[15px] text-text-main">
          {t(`share_claim_${state}`)}
        </p>
        {state === "claimed" ? (
          <button
            type="button"
            className="rounded-md bg-primary-blue px-5 py-2.5 text-[15px] text-white"
            onClick={() => navigate("/workspace/drive", { replace: true })}
          >
            {t("view_drive")}
          </button>
        ) : null}
      </div>
    </PageLayout>
  );
}
