import { Layout } from '../components/templates/Layout';
import { PageHeader } from '../components/atoms/PageHeader';
import { usePageTitle } from '../hooks/usePageTitle';
import { Dividend } from './Receipt/Dividend';
import { DomesticStock } from './Receipt/DomesticStock';
import { Mutualfund } from './Receipt/Mutualfund';
import { ConfirmDeleteModal } from '@/components/molecules/ConfirmDeleteModal/ConfirmDeleteModal';
import { ReceiptsTabNav } from '@/features/receipt/components/ReceiptsTabNav';
import { ReceiptsUtilityRail } from '@/features/receipt/components/ReceiptsUtilityRail';
import { useReceiptsState } from '@/features/receipt/hooks/useReceiptsState';

/**
 * 明細種類ごとにCSVデータを管理するページコンポーネント
 */
export function ReceiptsPage() {
  usePageTitle('取引明細');

  const {
    receiptsType,
    counts,
    tablistRef,
    setReceiptsType,
    handleTabKeyDown,
    showDeleteConfirm,
    closeDeleteConfirm,
    confirmDeleteAll,
    tabName,
    dbDataCount,
    data,
    summaries,
    previewData,
    utilityRailProps,
    deleteModalLoading,
    initialLoading,
    workspaceBusy,
  } = useReceiptsState();

  const utilityRail = <ReceiptsUtilityRail {...utilityRailProps} />;
  const panels = [
    {
      type: 'dividend',
      content: (
        <Dividend
          data={data.dividend}
          previewData={previewData.dividend}
          summary={summaries.dividend}
          utilityRail={utilityRail}
        />
      ),
    },
    {
      type: 'domesticstock',
      content: (
        <DomesticStock
          data={data.domesticstock}
          previewData={previewData.domesticstock}
          summary={summaries.domesticstock}
          utilityRail={utilityRail}
        />
      ),
    },
    {
      type: 'mutualfund',
      content: (
        <Mutualfund
          data={data.mutualfund}
          previewData={previewData.mutualfund}
          summary={summaries.mutualfund}
          utilityRail={utilityRail}
        />
      ),
    },
  ] as const;

  return (
    <Layout>
      <PageHeader
        title="取引明細"
        eyebrow="Transactions"
        description="配当金・国内株式・投資信託の取引明細を管理します。"
      />
      <ReceiptsTabNav
        receiptsType={receiptsType}
        tablistRef={tablistRef}
        onTabChange={setReceiptsType}
        onKeyDown={handleTabKeyDown}
        counts={counts}
      />
      <div className="mt-0" aria-busy={workspaceBusy}>
        <div data-testid="receipts-workspace">
          {!initialLoading &&
            panels.map((panel) => (
              <div
                key={panel.type}
                id={`tabpanel-${panel.type}`}
                role="tabpanel"
                aria-labelledby={`tab-${panel.type}`}
                hidden={receiptsType !== panel.type}
              >
                {receiptsType === panel.type && panel.content}
              </div>
            ))}
        </div>

        <ConfirmDeleteModal
          isOpen={showDeleteConfirm}
          onConfirm={confirmDeleteAll}
          onCancel={closeDeleteConfirm}
          title={`${tabName}データの全件削除`}
          description={`【${tabName}】のデータをすべて削除します。`}
          itemCount={dbDataCount}
          loading={deleteModalLoading}
        />
      </div>
    </Layout>
  );
}
