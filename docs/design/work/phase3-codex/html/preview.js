const params = new URLSearchParams(location.search);
const concept = params.get('concept') === 'c' ? 'c' : 'a';
const screen = ['home', 'asset', 'receipts'].includes(params.get('screen')) ? params.get('screen') : 'home';
const size = params.get('size') === 'mobile' ? 'mobile' : 'pc';
document.body.dataset.concept = concept;
document.body.dataset.size = size;
document.querySelectorAll('.concept').forEach((section, index) => {
  section.classList.toggle('selected', (index === 0 ? 'a' : 'c') === concept);
});
document.querySelectorAll('.screen').forEach(node => {
  const match = node.classList.contains(size) && node.classList.contains(screen);
  node.style.display = match ? 'block' : 'none';
});
