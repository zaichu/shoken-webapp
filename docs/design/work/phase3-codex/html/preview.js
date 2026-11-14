const params = new URLSearchParams(location.search);
const concept = params.get('concept') === 'c' ? 'c' : 'a';
const screen = ['home', 'asset', 'receipts'].includes(params.get('screen')) ? params.get('screen') : 'home';
const size = params.get('size') === 'mobile' ? 'mobile' : 'pc';
const theme = params.get('theme') === 'dark' ? 'dark' : 'light';
document.body.dataset.concept = concept;
document.body.dataset.screen = screen;
document.body.dataset.size = size;
document.body.dataset.theme = theme;
document.body.dataset.embedded = params.get('embedded') === '1';

const mobileFrame = document.querySelector('.mobile-viewport iframe');
if (params.get('embedded') !== '1') {
  const mobileUrl = new URL(location.href);
  mobileUrl.searchParams.set('size', 'mobile');
  mobileUrl.searchParams.set('embedded', '1');
  mobileFrame.src = mobileUrl.href;
}

for (const link of document.querySelectorAll('.preview-nav a')) {
  const url = new URL(link.href);
  const linkConcept = url.searchParams.get('concept');
  const linkScreen = url.searchParams.get('screen');
  const linkSize = url.searchParams.get('size');
  if (linkConcept === concept && linkScreen === screen && linkSize === size) {
    link.setAttribute('aria-current', 'page');
  }
}
