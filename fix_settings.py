import re
with open('src/components/SettingsView.vue', 'r', encoding='gbk') as f:
    content = f.read()

content = re.sub(
    r'const\s*\{\s*editUsername,\s*editAvatarId,\s*editAvatarBase64,\s*updateProfile,\s*selectAndUploadAvatar,\s*\}\s*=\s*useSettings\(\);',
    '''const { 
  editUsername, editAvatarId, editAvatarBase64, 
  chatFontSize, globalFontSize, isDarkTheme, defaultRenderLatex, appAccentColor,
  updateProfile, selectAndUploadAvatar, 
  saveFontSize, saveGlobalFontSize, saveDefaultRenderLatex, toggleTheme, setAccentColor
} = useSettings();''',
    content
)

with open('src/components/SettingsView.vue', 'w', encoding='gbk') as f:
    f.write(content)
