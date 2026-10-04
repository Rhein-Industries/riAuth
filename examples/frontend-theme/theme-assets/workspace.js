"use strict";
// Trusted operator UI code; existing auth/API modules keep their own behavior.
const welcome = document.getElementById("theme-welcome");
if (welcome) {
  welcome.textContent = "Make yourself at home.";
  welcome.hidden = false;
}

// The embedded app updates its title when switching applications/favorites.
// Keep that page name while applying this deployment's title branding.
const title = document.querySelector("title");
if (title) {
  const brandTitle = () => {
    const branded = document.title.replace(/ · riAuth$/, " · Acme workspace");
    if (document.title !== branded) document.title = branded;
  };
  new MutationObserver(brandTitle).observe(title, {
    childList: true, characterData: true, subtree: true
  });
  brandTitle();
}
