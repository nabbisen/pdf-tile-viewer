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
      width: calc(100% - 3.6rem);
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
    left: 4.1rem;
    bottom: 0.5rem;
    height: 0.8rem;
    background-color: #fff49132;
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
    color: #636363;
  }
  h1 span:nth-child(2) {
    color: #878787;
  }
  h1 span:nth-child(3) {
    color: #939393;
  }
</style>
