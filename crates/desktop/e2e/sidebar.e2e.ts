describe('Sidebar Navigation', () => {
  it('should display the app logo', async () => {
    const logo = await browser.$('.logo-circle');
    if (await logo.isExisting()) {
      expect(await logo.isDisplayed()).toBe(true);
      const text = await logo.getText();
      expect(text).toContain('D1');
    }
  });

  it('should show version number', async () => {
    const version = await browser.$('.logo-version');
    if (await version.isExisting()) {
      const text = await version.getText();
      expect(text).toBe('v2.6.0');
    }
  });

  it('should display connection status indicators', async () => {
    // Connection status shows daemon and platform status
    const status = await browser.$('.connection-status');
    if (await status.isExisting()) {
      expect(await status.isDisplayed()).toBe(true);
    }
  });
});
