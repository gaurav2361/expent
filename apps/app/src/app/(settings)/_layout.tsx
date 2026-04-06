import { Stack } from 'expo-router';

export default function SettingsLayout() {
  return (
    <Stack>
      <Stack.Screen name="account" options={{ title: 'Account' }} />
      <Stack.Screen name="profile" options={{ title: 'Profile' }} />
      <Stack.Screen name="appearance" options={{ title: 'Appearance' }} />
    </Stack>
  );
}
