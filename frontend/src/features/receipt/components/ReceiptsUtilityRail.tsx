import { Spinner } from '@/components/atoms/Spinner';
import {
  DataActionRail,
  type DataActionRailProps,
} from '@/components/organisms/DataActionRail/DataActionRail';
import type { CsvPreview } from '../reducer';
import { ReceiptsAlerts } from './ReceiptsAlerts';

export interface ReceiptsUtilityRailProps {
  actionRailProps: DataActionRailProps;
  alertsProps: {
    dbError: string | null | undefined;
    hasCsvFile: boolean;
    previewing: boolean;
    csvPreview: CsvPreview | null | undefined;
  };
  authLoading: boolean;
  dbLoading: boolean;
}

export function ReceiptsUtilityRail({
  actionRailProps,
  alertsProps,
  authLoading,
  dbLoading,
}: ReceiptsUtilityRailProps) {
  return (
    <>
      <DataActionRail {...actionRailProps} />
      <ReceiptsAlerts {...alertsProps} />
      <div aria-live="polite" aria-atomic="true">
        {(authLoading || dbLoading) && (
          <section className="px-5 py-4" role="status">
            <div className="status-message">
              <Spinner size="md" className="text-primary" />
              <p className="text-sm text-secondary">
                {authLoading && '認証状態を確認しています...'}
                {dbLoading && 'データを読み込んでいます...'}
              </p>
            </div>
          </section>
        )}
      </div>
    </>
  );
}
