const params = new URLSearchParams(location.search);
const concept = params.get('concept') === 'c' ? 'c' : 'a';
const screen = ['home', 'asset', 'receipts'].includes(params.get('screen')) ? params.get('screen') : 'home';
const size = params.get('size') === 'mobile' ? 'mobile' : 'pc';
const theme = params.get('theme') === 'dark' ? 'dark' : 'light';
document.body.dataset.concept = concept;
document.body.dataset.screen = screen;
document.body.dataset.size = size;
document.body.dataset.theme = theme;
