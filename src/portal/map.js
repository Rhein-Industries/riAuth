"use strict";
(() => {
  const base = document.querySelector("meta[name=riauth-base]").content;
  const svg = document.getElementById("event-map");
  const ns = svg.namespaceURI;
  const status = document.getElementById("map-status");
  const unknown = document.getElementById("unknown-value");
  const list = document.getElementById("point-list");

  function say(text) {
    status.textContent = text;
  }

  function plot(points) {
    const layer = document.getElementById("event-points");
    layer.replaceChildren();
    list.replaceChildren();
    for (const point of points) {
      if (typeof point.latitude !== "number" || typeof point.longitude !== "number" || typeof point.count !== "number") {
        continue;
      }
      const circle = document.createElementNS(ns, "circle");
      circle.setAttribute("class", "event-point");
      circle.setAttribute("cx", String(point.longitude + 180));
      circle.setAttribute("cy", String(90 - point.latitude));
      circle.setAttribute("r", String(Math.min(6, 0.9 + Math.sqrt(point.count))));
      const name = typeof point.label === "string" && point.label ? point.label + ", " : "";
      const title = document.createElementNS(ns, "title");
      title.textContent = name + point.count + " events";
      circle.append(title);
      layer.append(circle);
      const item = document.createElement("li");
      item.textContent = name + point.latitude.toFixed(1) + ", " + point.longitude.toFixed(1) + " — " + point.count;
      list.append(item);
    }
  }

  async function load(event) {
    if (event) event.preventDefault();
    const since = document.getElementById("since").value.trim();
    const until = document.getElementById("until").value.trim();
    const action = document.getElementById("action-prefix").value.trim();
    if (since && until && Number(since) > Number(until)) {
      say("The time range ends before it starts.");
      return;
    }
    const params = new URLSearchParams();
    if (since) params.set("since", since);
    if (until) params.set("until", until);
    if (action) params.set("action", action);
    const query = params.toString();
    say("Loading stored events…");
    try {
      const response = await fetch(base + "api/audit/map" + (query ? "?" + query : ""), {
        method: "GET",
        credentials: "same-origin",
        mode: "same-origin",
        cache: "no-store",
        referrerPolicy: "same-origin",
        headers: { "Accept": "application/json" }
      });
      if (response.status === 401) {
        say("Sign in as an administrator from your applications, then open this page again.");
        return;
      }
      if (response.status === 403) {
        say("This map requires audit.read. It is not part of the application launcher.");
        return;
      }
      if (!response.ok) {
        say("The map request was rejected.");
        return;
      }
      const body = await response.json();
      const points = Array.isArray(body.points) ? body.points : [];
      const unknownCount = typeof body.unknown === "number" ? body.unknown : 0;
      plot(points);
      unknown.textContent = String(unknownCount);
      const notes = [points.length + " areas, " + unknownCount + " events without coordinates."];
      if (body.truncated) notes.push("Scanning stopped at the event limit. Narrow the time range to see older events.");
      if (body.omitted_cells) notes.push(body.omitted_cells + " additional areas were omitted.");
      say(notes.join(" "));
    } catch {
      say("The map could not be loaded from this server.");
    }
  }

  document.getElementById("map-filters").addEventListener("submit", load);
  load();
})();
