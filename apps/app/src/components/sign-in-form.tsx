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

export function SignInForm() {
  const [email, setEmail] = React.useState('');
  const [password, setPassword] = React.useState('');
  const { signIn, isLoading } = useAuth();
  const passwordInputRef = React.useRef<TextInput>(null);

  function onEmailSubmitEditing() {
    passwordInputRef.current?.focus();
  }

  async function onSubmit() {
    if (!email || !password) {
      showErrorMessage('Please enter both email and password');
      return;
    }

    try {
      await signIn(email, password);
      router.replace('/(tabs)');
    } catch (error) {
      showErrorMessage(error instanceof Error ? error.message : 'Failed to sign in');
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
          <CardTitle className="text-2xl font-bold">Welcome back</CardTitle>
          <CardDescription className="text-muted-foreground text-center">
            Enter your details to access your account
          </CardDescription>
        </CardHeader>
        <CardContent className="gap-6 pt-4">
          <View className="gap-5">
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
              <View className="flex-row items-center justify-between px-1">
                <Label nativeID="password-label" className="font-semibold">Password</Label>
                <Pressable
                  onPress={() => {
                    router.push('/(auth)/forgot-password');
                  }}>
                  <Text className="text-primary text-xs font-bold">Forgot?</Text>
                </Pressable>
              </View>
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
                {isLoading ? 'Signing in...' : 'Sign In'}
              </Text>
            </Button>
          </View>

          <View className="flex-row items-center py-2">
            <Separator className="flex-1 opacity-50" />
            <Text className="text-muted-foreground px-4 text-xs font-medium uppercase tracking-widest">or continue with</Text>
            <Separator className="flex-1 opacity-50" />
          </View>

          <SocialConnections />

          <View className="flex-row justify-center gap-1.5 mt-4">
            <Text className="text-muted-foreground">New to Expent?</Text>
            <Pressable
              onPress={() => {
                router.push('/(auth)/sign-up');
              }}>
              <Text className="font-bold text-primary">Create account</Text>
            </Pressable>
          </View>
        </CardContent>
      </Card>
    </Animated.View>
  );
}
