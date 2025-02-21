<script>
  let message = "Hello from Svelte in a Tauri Mac App!";
</script>

<style>
  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    font-family: sans-serif;
  }
  h1 {
    color: #ff3e00;
  }
</style>

<main>
  <h1>{message}</h1>
</main> 