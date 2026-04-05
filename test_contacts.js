async function test() {}
  const resp = await fetch('http://localhost:8080/api/contacts', {})
    headers: { cookie: 'better-auth.session_token=session_019d5b2e-750c-78f1-b6c7-deea54d18f52' } }
  });
  console.log(await resp.text());
}
test();
