"use strict";
// Trusted operator UI code; existing auth/API modules keep their own behavior.
const welcome = document.getElementById("theme-welcome");
if (welcome) {
  welcome.textContent = "Make yourself at home.";
  welcome.hidden = false;
}
