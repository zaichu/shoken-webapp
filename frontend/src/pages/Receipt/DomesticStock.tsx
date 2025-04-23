
import { ReceiptTemplate } from '@/components';
import React from 'react';

interface DomesticStockProps {
    csvData: any[];
}

const Header = () => {
    return (
        <div className="card mt-2">
            <div className="card-header bg-primary text-white">
                <h5 className="mb-0">集計情報</h5>
            </div>
            <div className="card-body">
                <div className="row">
                    <div className="col-md-4">
                        <h6>配当金合計</h6>
                        {/* <p className="h4">{calculations.formattedTotals.totalDividends}</p> */}
                    </div>
                    <div className="col-md-4">
                        <h6>税額合計</h6>
                        {/* <p className="h4">{calculations.formattedTotals.totalTaxes}</p> */}
                    </div>
                    <div className="col-md-4">
                        <h6>受取金額合計</h6>
                        {/* <p className="h4">{calculations.formattedTotals.totalNetAmount}</p> */}
                    </div>
                </div>
            </div>
        </div>
    )
}

export const DomesticStock: React.FC<DomesticStockProps> = ({ csvData }: DomesticStockProps) => {
    return (
        <ReceiptTemplate title="国内株式">
            <div className="table-responsive">
                <table className="table table-striped mb-0">
                    <thead className="bg-light">
                        <tr>
                            <th>入金日</th>
                            <th>銘柄コード</th>
                            <th>銘柄名</th>
                            <th>単価</th>
                            <th>数量</th>
                            <th>受取金額</th>
                        </tr>
                    </thead>
                    <tbody>
                        {/* {filteredData.map((item, index) => (
                            <tr key={index}>
                                <td>{item['入金日']}</td>
                                <td>{item['銘柄コード']}</td>
                                <td>{item['銘柄']}</td>
                                <td>{item['単価[円/現地通貨]']}</td>
                                <td>{item['数量[株/口]']}</td>
                                <td>{item['受取金額[円/現地通貨]']}</td>
                            </tr>
                        ))} */}
                    </tbody>
                </table>
            </div>
        </ReceiptTemplate>
    );
};
