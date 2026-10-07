"use strict";
// This small script runs before stylesheets so a saved choice applies before paint.
(() => {
  const base = document.querySelector('meta[name="riauth-base"]').content;
  const key = `riauth:appearance:${base}`;
  const system = matchMedia("(prefers-color-scheme: dark)");
  const valid = (value) => ["light", "dark", "system"].includes(value) ? value : "system";
  let choice = "system";
  try { choice = valid(localStorage.getItem(key)); } catch { /* Storage is optional. */ }
  function apply() {
    document.documentElement.dataset.theme = choice === "system" ? system.matches ? "dark" : "light" : choice;
    for (const control of document.querySelectorAll("[data-appearance]")) control.value = choice;
    const meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.content = document.documentElement.dataset.theme === "dark" ? "#111522" : "#f7f8fc";
  }
  function set(value) {
    choice = valid(value);
    try { localStorage.setItem(key, choice); } catch { /* Keep the choice in this tab. */ }
    apply();
  }
  system.addEventListener("change", apply);
  window.addEventListener("storage", (event) => {
    if (event.key === key || event.key === null) { choice = valid(event.key === null ? null : event.newValue); apply(); }
  });
  document.addEventListener("DOMContentLoaded", () => {
    for (const control of document.querySelectorAll("[data-appearance]")) control.addEventListener("change", () => set(control.value));
    apply();
  });
  window.RiAuthAppearance = Object.freeze({ get: () => choice, set });
  apply();
})();
