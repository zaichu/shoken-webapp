function showTab(tabName) {
    const tabs = document.querySelectorAll('.tab');
    const tabContents = document.querySelectorAll('.tab-content');

    tabs.forEach(tab => tab.classList.remove('active'));
    tabContents.forEach(content => content.classList.remove('active'));

    document.querySelector(`.tab[id="${tabName}-a"]`).classList.add('active');
    document.getElementById(`${tabName}-div`).classList.add('active');
}

function processCSV(file, type) {
    const formData = new FormData();
    formData.append('file', file);

    fetch(`/process-csv/${type}`, {
        method: 'POST',
        body: formData
    })
        .then(response => response.text())
        .then(html => {
            document.getElementById(`${type}-result-container`).innerHTML = html;
        })
        .catch(error => {
            console.error('Error:', error);
            alert('エラーが発生しました。');
        });
}

document.addEventListener('DOMContentLoaded', () => {
    const dividendInput = document.getElementById('dividend-csv');
    const profitLossInput = document.getElementById('profit-loss-csv');

    dividendInput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (file) {
            processCSV(file, 'dividend');
        }
    });

    profitLossInput.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (file) {
            processCSV(file, 'profit-loss');
        }
    });
});
