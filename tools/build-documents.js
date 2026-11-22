#!/usr/bin/env node
/* Build documents.html for Flynn's Arcade from the hub page itself:
   extract the CABS array out of dist/index.html and render every
   cabinet's story, lesson, how-to-play and house note as one page. */
const fs = require('fs');
const path = require('path');

const dist = path.join(__dirname, '..', 'dist');
const hub = fs.readFileSync(path.join(dist, 'index.html'), 'utf8');

const start = hub.indexOf('var CABS = ');
if (start < 0) { console.error('CABS not found'); process.exit(1); }
let i = hub.indexOf('[', start), depth = 0, inStr = false, escaped = false, end = -1;
for (let j = i; j < hub.length; j++) {
  const ch = hub[j];
  if (inStr) {
    if (escaped) escaped = false;
    else if (ch === '\\') escaped = true;
    else if (ch === '"') inStr = false;
  } else if (ch === '"') inStr = true;
  else if (ch === '[') depth++;
  else if (ch === ']') { depth--; if (depth === 0) { end = j; break; } }
}
if (end < 0) { console.error('CABS array not closed'); process.exit(1); }
const CABS = eval(hub.slice(i, end + 1));

const PAGES = {
  asteroids: 'games/asteroids.html', tron: 'games/tron.html', invaders: 'games/invaders.html',
  defender: 'games/defender.html', airplane: 'games/paper-airplane.html', soup: 'games/primordial-soup.html',
  bitbyte: 'games/robot-daemon-scroller.html', pinball: 'games/pinball.html', breakout: 'games/breakout.html',
  tempest: 'games/tempest.html', frogger: 'games/frogger.html', karts: 'games/ai-karts.html',
  solitaire: 'games/solitaire.html', tarot: 'games/tarot.html', bulletin: 'games/bulletin-board.html',
  crossroads: 'games/crossroads.html', infinity: 'games/infinity-complex.html',
  wandering: 'games/wandering-hub.html', gravitygolf: 'games/gravity-golf.html'
};
const esc = s => String(s == null ? '' : s).replace(/&/g, '&amp;').replace(/</g, '&lt;');

const cards = CABS.map((c, n) => `
  <article>
    <h2>${n + 1}. ${esc(c.name)}</h2>
    <p class="era">${esc(c.yr)} · ${esc((c.tags || []).join(' · '))}</p>
    <h3>The game</h3><p>${esc(c.game)}</p>
    <h3>The lesson — taught by doing</h3><p>${esc(c.lessonLong)}</p>
    <h3>How you play it</h3><p>${esc(c.play)}</p>
    <p class="note"><b>House note —</b> ${esc(String(c.note || '').replace(/<[^>]*>/g, ''))}</p>
    ${PAGES[c.id] ? `<p><a class="play" href="${PAGES[c.id]}">▶ Play this cabinet</a></p>` : '<p class="note">A reading cabinet — this one is played right here in the room.</p>'}
  </article>`).join('\n');

const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Cabinet Documents — Flynn's Arcade</title>
<style>
  body { margin: 0; background: #14101e; color: #f4efe4; font-family: "Segoe UI", system-ui, sans-serif; line-height: 1.55; }
  .wrap { max-width: 860px; margin: 0 auto; padding: 32px 22px 72px; }
  h1 { font-family: Georgia, serif; font-size: clamp(32px, 5vw, 48px); margin: 8px 0 6px; }
  .sub { color: #bdb4d6; margin: 0 0 28px; }
  article { background: #1d1830; border: 1px solid #372e55; border-radius: 14px; padding: 22px 24px; margin: 0 0 20px; }
  h2 { margin: 0 0 4px; font-size: 23px; }
  h3 { margin: 16px 0 4px; font-size: 13px; letter-spacing: .12em; text-transform: uppercase; color: #7ef0d4; }
  p { margin: 0 0 6px; }
  .era { color: #bdb4d6; font-size: 14px; }
  .note { color: #d8d2ea; }
  a.play { display: inline-block; margin-top: 10px; background: #7ef0d4; color: #14101e; font-weight: 700; padding: 10px 18px; border-radius: 999px; text-decoration: none; }
  a.back { color: #7ef0d4; font-weight: 600; }
</style>
</head>
<body>
<div class="wrap">
  <p><a class="back" href="index.html">← Back to the arcade</a></p>
  <h1>Cabinet Documents</h1>
  <p class="sub">Every cabinet in Flynn's Arcade — its story, the lesson it teaches by doing, and how to play. Generated from the room itself, so it can never drift out of date.</p>
  ${cards}
</div>
</body>
</html>
`;
fs.writeFileSync(path.join(dist, 'documents.html'), html);
console.log(`documents.html built: ${CABS.length} cabinets`);
