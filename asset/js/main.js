document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('upload-form');
    const fileInput = document.getElementById('csv-file');
    const resultContainer = document.getElementById('result');

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
});