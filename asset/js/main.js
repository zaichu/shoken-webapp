document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('upload-form');
    const fileInput = document.getElementById('csv-file');
    const resultContainer = document.getElementById('result-container');

    form.addEventListener('submit', async (e) => {
        e.preventDefault();

        const formData = new FormData();
        formData.append('file', fileInput.files[0]);

        try {
            const response = await fetch('/upload', {
                method: 'POST',
                body: formData
            });

            if (response.ok) {
                const result = await response.text();
                resultContainer.innerHTML = result;
            } else {
                throw new Error('Upload failed');
            }
        } catch (error) {
            console.error('Error:', error);
            resultContainer.innerHTML = '<p>An error occurred while uploading the file.</p>';
        }
    });

    fileInput.addEventListener('change', () => {
        const fileName = fileInput.files[0]?.name || 'Choose a CSV file';
        fileInput.nextElementSibling.textContent = fileName;
    });

    document.getElementById('process-dividend').addEventListener('click', () => processCSV('dividend'));
    document.getElementById('process-profit-loss').addEventListener('click', () => processCSV('profit-loss'));

    function processCSV(type) {
        const file = fileInput.files[0];
        if (!file) {
            alert('ファイルを選択してください。');
            return;
        }

        const formData = new FormData();
        formData.append('file', file);

        fetch(`/process-csv/${type}`, {
            method: 'POST',
            body: formData
        })
            .then(response => response.text())
            .then(html => {
                resultContainer.innerHTML = html;
            })
            .catch(error => {
                console.error('Error:', error);
                alert('エラーが発生しました。');
            });
    }
});
