describe('App Launch', () => {
  it('should launch the application', async () => {
    // Wait for the app to load
    const title = await browser.getTitle();
    expect(title).toBe('Day1 Doctor');
  });

  it('should display the main window', async () => {
    // Check that the app root element exists
    const app = await browser.$('#app');
    await app.waitForExist({ timeout: 10000 });
    expect(await app.isExisting()).toBe(true);
  });

  it('should show login screen or main UI', async () => {
    // The app either shows login or the main interface
    const body = await browser.$('body');
    await body.waitForExist({ timeout: 10000 });
    const text = await body.getText();
    // Should contain some recognizable Day1 Doctor content
    const hasContent =
      text.includes('Day1') ||
      text.includes('Dr.') ||
      text.includes('Login') ||
      text.includes('Sign in');
    expect(hasContent).toBe(true);
  });
});

describe('Window Properties', () => {
  it('should have correct minimum dimensions', async () => {
    const size = await browser.getWindowSize();
    // Minimum is 900x600 per tauri.conf.json
    expect(size.width).toBeGreaterThanOrEqual(900);
    expect(size.height).toBeGreaterThanOrEqual(600);
  });
});
