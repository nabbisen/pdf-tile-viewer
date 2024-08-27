<header>
  <div class="logo">
    <img src="/logo.png" alt="" />
    <h1><span>PDF</span> <span>Tile</span> <span>Viewer</span></h1>
  </div>
</header>

<style>
  header {
    position: relative;
    z-index: 10000;
  }

  @keyframes logo-bg {
    from {
      width: 4.4rem;
    }
    to {
      width: calc(100% + 0.6rem);
    }
  }
  .logo {
    position: relative;
    width: fit-content;
    display: flex;
  }
  .logo::before {
    content: '';
    position: absolute;
    width: 4.4rem;
    left: 0;
    bottom: -0.2rem;
    height: 1.9rem;
    background-color: #bdd4ff27;
    animation: logo-bg 4.4s ease forwards;
  }
  img {
    height: 3.2rem;
    margin: 0.2rem 0.6rem;
  }
  img:hover {
    transform: translate(0.05rem, 0.05rem);
    transition: transform 0.2s ease;
  }
  h1 {
    font-size: 1.8rem;
    padding: 0;
    margin: 0;
    display: flex;
    align-items: center;
  }
  h1 span:not(:first-child) {
    margin-left: 0.3rem;
  }
  h1 span {
    color: #002872;
  }
  h1 span:nth-child(2) {
    color: #00073e;
  }
  h1 span:nth-child(3) {
    color: #00031c;
  }
</style>
