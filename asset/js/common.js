function toggleMenu() {
    const menu = document.querySelector('.menu');
    menu.classList.toggle('active');
}

function initializeHamburgerMenu() {
    const hamburgerButton = document.querySelector('.hamburger-icon');
    if (hamburgerButton) {
        hamburgerButton.addEventListener('click', toggleMenu);
    }
}
