describe('Connection Status', () => {
  it('should show daemon status', async () => {
    // Look for the connection status component
    const statusElements = await browser.$$('[class*="status"]');
    expect(statusElements.length).toBeGreaterThan(0);
  });

  it('should show platform connectivity info', async () => {
    const body = await browser.$('body');
    await body.waitForExist({ timeout: 10000 });
    // The app should display some connection-related UI
    const text = await body.getText();
    const hasConnectionInfo =
      text.includes('online') ||
      text.includes('offline') ||
      text.includes('connecting') ||
      text.includes('Daemon') ||
      text.includes('Platform');
    expect(hasConnectionInfo).toBe(true);
  });
});
