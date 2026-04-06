import { View } from 'react-native';
import { Text } from '@/components/ui/text';

export default function ContactsScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Contacts</Text>
      <Text className="text-muted-foreground mt-2">Manage your friends and business contacts.</Text>
    </View>
  );
}
