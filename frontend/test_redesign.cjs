const puppeteer = require('/home/gulshansharma/Proofer/frontend/node_modules/puppeteer-core');
const fs = require('fs');

(async () => {
  console.log('Launching browser for Proofer Workstation Visual & Interaction Critique...');
  const browser = await puppeteer.launch({
    executablePath: '/usr/bin/google-chrome',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
    defaultViewport: { width: 1440, height: 900, deviceScaleFactor: 2 },
  });

  const page = await browser.newPage();
  await page.goto('http://127.0.0.1:8086', { waitUntil: 'networkidle0', timeout: 10000 });
  await new Promise(r => setTimeout(r, 1000));

  const outDir = '/home/gulshansharma/.gemini/antigravity/brain/09d92c5e-483d-4ba4-adc0-f985b4f3c162';

  // 1. Full Workstation Default Screenshot (Isosceles)
  await page.screenshot({ path: `${outDir}/workstation_default.png` });
  console.log('Saved workstation_default.png');

  // 2. Select Step 2 in the Execution Trace to test 3-Way Sync
  const steps = await page.$$('.trace-step-item');
  if (steps.length >= 2) {
    await steps[1].click();
    await new Promise(r => setTimeout(r, 600));
    await page.screenshot({ path: `${outDir}/step_selection_sync.png` });
    console.log('Saved step_selection_sync.png');
  }

  // 3. Open Command Palette (Ctrl+K)
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyK');
  await page.keyboard.up('Control');
  await new Promise(r => setTimeout(r, 600));
  await page.screenshot({ path: `${outDir}/command_palette.png` });
  console.log('Saved command_palette.png');

  // Close Command Palette with Escape
  await page.keyboard.press('Escape');
  await new Promise(r => setTimeout(r, 400));

  // 4. Open Version Control Drawer to inspect Proof State Diff
  const vcsBtn = await page.$('.menu-nav-btn:nth-child(2)');
  if (vcsBtn) {
    await vcsBtn.click();
    await new Promise(r => setTimeout(r, 600));
    await page.screenshot({ path: `${outDir}/vcs_proof_diff.png` });
    console.log('Saved vcs_proof_diff.png');
    // Close VCS
    const closeBtn = await page.$('.drawer-close-btn');
    if (closeBtn) await closeBtn.click();
    await new Promise(r => setTimeout(r, 400));
  }

  // 5. Switch to Thales' Circle Theorem
  await page.select('#example-select', 'thales');
  await new Promise(r => setTimeout(r, 1000));
  await page.screenshot({ path: `${outDir}/thales_circle_workstation.png` });
  console.log('Saved thales_circle_workstation.png');

  // 6. Switch to Pure Propositional Logic (P -> P) to verify canvas auto-collapse
  await page.select('#example-select', 'logic_identity');
  await new Promise(r => setTimeout(r, 1000));
  await page.screenshot({ path: `${outDir}/pure_logic_collapsed_canvas.png` });
  console.log('Saved pure_logic_collapsed_canvas.png');

  await browser.close();
  console.log('All automated verification screenshots completed successfully!');
})().catch(err => {
  console.error('Error during test:', err);
  process.exit(1);
});
