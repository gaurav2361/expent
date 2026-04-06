import { View } from 'react-native';
import { Text } from '@/components/ui/text';

export default function TransactionsScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold">Transactions</Text>
      <Text className="text-muted-foreground mt-2">Historical transaction data will appear here.</Text>
    </View>
  );
}
