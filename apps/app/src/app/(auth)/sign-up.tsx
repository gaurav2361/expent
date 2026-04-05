import { View } from 'react-native';
import { Text } from '@/components/ui/text';
import { Button } from '@/components/ui/button';
import { Link } from 'expo-router';

export default function SignUpScreen() {
  return (
    <View className="flex-1 items-center justify-center bg-background p-6">
      <Text className="text-2xl font-bold mb-4">Create Account</Text>
      <Link href="/(auth)/sign-in" asChild>
        <Button variant="ghost">
          <Text>Already have an account? Sign In</Text>
        </Button>
      </Link>
    </View>
  );
}
