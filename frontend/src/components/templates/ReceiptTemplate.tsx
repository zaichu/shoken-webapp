import React, { ReactNode } from 'react';
import { SearchCard } from '@/components/organisms/SearchCard/SearchCard';
import { Card, CardBody } from '@/components/atoms/Card';
import { WorkspaceShell } from '@/components/templates/WorkspaceShell';
import { SearchCategories } from '@/types/common';

interface ReceiptTemplateProps {
  title?: string;
  header?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  onSearch?: (query: string) => void;
  searchCategories?: SearchCategories;
  onSearchExpandToggle?: (isExpanded: boolean) => void;
  layout?: 'stack' | 'workspace';
  utilityRail?: ReactNode;
}

const ReceiptTemplateContent: React.FC<ReceiptTemplateProps> = ({
  header,
  children,
  footer,
  onSearch,
  searchCategories,
  onSearchExpandToggle,
  layout: layoutProp,
  utilityRail,
}) => {
  const layout = layoutProp ?? (utilityRail ? 'workspace' : 'stack');
  const handleSearchExpandToggle = (isExpanded: boolean) => {
    onSearchExpandToggle?.(isExpanded);
  };

  const searchCard = onSearch ? (
    <SearchCard
      onSearch={onSearch}
      categories={searchCategories}
      onExpandToggle={handleSearchExpandToggle}
      initialExpanded={layout === 'workspace'}
      compact={layout === 'workspace'}
    />
  ) : null;

  const mainCard = (
    <Card className="overflow-hidden border-slate-950/10 bg-white/95 shadow-[0_16px_44px_-36px_rgba(15,23,42,0.9)]" data-testid="receipt-card">
      <CardBody className="p-0" data-testid="receipt-card-body">
        {children}
      </CardBody>
    </Card>
  );

  if (layout === 'workspace') {
    return (
      <div className="space-y-1.5" data-testid="receipt-container">
        <WorkspaceShell
          testIdPrefix="receipt"
          mainClassName="space-y-2"
          main={<>{header}{mainCard}</>}
          rail={<>{utilityRail}{searchCard}</>}
        />
        {footer ? <div>{footer}</div> : null}
      </div>
    );
  }

  return (
    <div className="space-y-1.5" data-testid="receipt-container">
      {searchCard}
      {header ? <div>{header}</div> : null}
      {mainCard}
      {footer ? <div>{footer}</div> : null}
    </div >
  );
};

export const ReceiptTemplate: React.FC<ReceiptTemplateProps> = ReceiptTemplateContent;
