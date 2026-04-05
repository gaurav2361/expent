import { SocialConnections } from '@/components/social-connections';
import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Separator } from '@/components/ui/separator';
import { Text } from '@/components/ui/text';
import * as React from 'react';
import { Pressable, type TextInput, View } from 'react-native';
import { useAuth } from '@/lib/auth/use-auth';
import { router } from 'expo-router';
import { showErrorMessage } from '@/components/ui/utils';
import Animated, { FadeIn } from 'react-native-reanimated';

export function SignUpForm() {
  const [name, setName] = React.useState('');
  const [email, setEmail] = React.useState('');
  const [password, setPassword] = React.useState('');
  const { signUp, isLoading } = useAuth();
  const passwordInputRef = React.useRef<TextInput>(null);

  function onEmailSubmitEditing() {
    passwordInputRef.current?.focus();
  }

  async function onSubmit() {
    if (!name || !email || !password) {
      showErrorMessage('Please fill in all fields');
      return;
    }

    try {
      await signUp(email, password, name);
      router.replace('/(tabs)');
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : 'Failed to create account');
    }
  }

  return (
    <Animated.View entering={FadeIn.duration(500)} className="gap-8">
      {/* Header Branding */}
      <View className="items-center gap-2 mb-2">
        <View className="bg-primary w-14 h-14 rounded-2xl items-center justify-center shadow-md shadow-primary/20">
          <Text className="text-primary-foreground text-3xl font-bold tracking-tighter">E</Text>
        </View>
        <Text className="text-3xl font-bold text-foreground tracking-tight">Expent</Text>
      </View>

      <Card className="border-border/50 shadow-none bg-card/50 rounded-[32px] border">
        <CardHeader className="pb-2 items-center">
          <CardTitle className="text-2xl font-bold">Create account</CardTitle>
          <CardDescription className="text-muted-foreground text-center">
            Join Expent to start managing your finances
          </CardDescription>
        </CardHeader>
        <CardContent className="gap-6 pt-4">
          <View className="gap-5">
            <View className="gap-2">
              <Label nativeID="name-label" className="ml-1 font-semibold">Full Name</Label>
              <Input
                id="name"
                placeholder="John Doe"
                autoCapitalize="words"
                value={name}
                onChangeText={setName}
                returnKeyType="next"
                className="h-14 rounded-2xl px-4 bg-background border-border/60"
                aria-labelledby="name-label"
              />
            </View>
            <View className="gap-2">
              <Label nativeID="email-label" className="ml-1 font-semibold">Email</Label>
              <Input
                id="email"
                placeholder="name@example.com"
                keyboardType="email-address"
                autoComplete="email"
                autoCapitalize="none"
                value={email}
                onChangeText={setEmail}
                onSubmitEditing={onEmailSubmitEditing}
                returnKeyType="next"
                className="h-14 rounded-2xl px-4 bg-background border-border/60"
                aria-labelledby="email-label"
              />
            </View>
            <View className="gap-2">
              <Label nativeID="password-label" className="ml-1 font-semibold">Password</Label>
              <Input
                ref={passwordInputRef}
                id="password"
                placeholder="••••••••"
                secureTextEntry
                value={password}
                onChangeText={setPassword}
                returnKeyType="send"
                onSubmitEditing={onSubmit}
                className="h-14 rounded-2xl px-4 bg-background border-border/60"
                aria-labelledby="password-label"
              />
            </View>
            <Button className="w-full h-14 rounded-2xl bg-primary mt-2 shadow-lg shadow-primary/20" onPress={onSubmit} disabled={isLoading}>
              <Text className="text-primary-foreground font-bold text-lg">
                {isLoading ? 'Creating...' : 'Create Account'}
              </Text>
            </Button>
          </View>

          <View className="flex-row items-center py-2">
            <Separator className="flex-1 opacity-50" />
            <Text className="text-muted-foreground px-4 text-xs font-medium uppercase tracking-widest">or sign up with</Text>
            <Separator className="flex-1 opacity-50" />
          </View>

          <SocialConnections />

          <View className="flex-row justify-center gap-1.5 mt-4">
            <Text className="text-muted-foreground">Already have an account?</Text>
            <Pressable
              onPress={() => {
                router.push('/(auth)/sign-in');
              }}>
              <Text className="font-bold text-primary">Sign in</Text>
            </Pressable>
          </View>
        </CardContent>
      </Card>
    </Animated.View>
  );
}
