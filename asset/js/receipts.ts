function showTab(tabName: string): void {
    const tabs = document.querySelectorAll('.tab');
    const tabContents = document.querySelectorAll('.tab-content');

    tabs.forEach(tab => tab.classList.remove('active'));
    tabContents.forEach(content => content.classList.remove('active'));

    const tabElement = document.querySelector(`.tab[id="${tabName}-a"]`);
    if (tabElement)
        tabElement.classList.add('active');

    const tabContentElement = document.getElementById(`${tabName}-div`);
    if (tabContentElement)
        tabContentElement.classList.add('active');

}

function processCSV(file: File, type: string): void {
    const formData = new FormData();
    formData.append('file', file);

    fetch(`/process-csv/${type}`, {
        method: 'POST',
        body: formData
    })
        .then(response => response.text())
        .then(html => {
            const resultContainer = document.getElementById(`${type}-result-container`);
            if (resultContainer)
                resultContainer.innerHTML = html;
        })
        .catch(error => {
            console.error('Error:', error);
            alert('エラーが発生しました。');
        });
}

document.addEventListener('DOMContentLoaded', () => {
    const dividendInput = document.getElementById('dividend-csv');
    dividendInput?.addEventListener('change', (event) => {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (file) {
            processCSV(file, 'dividend');
        }
    });

    const profitLossInput = document.getElementById('profit-loss-csv');
    profitLossInput?.addEventListener('change', (event) => {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];
        if (file) {
            processCSV(file, 'profit-loss');
        }
    });
});
