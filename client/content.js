(async () => {
  const {
    default: init,
    start_websocket,
    send_location,
  } = await import(chrome.runtime.getURL("pkg/client.js"));
  //init, { start_websocket, send_location } =
  init().then(() => {
    let id_map = {};

    let move_cursor_cb = function (id, x, y) {
      if (id in id_map) {
        let element = id_map[id];

        let pixel_x = window.innerWidth * x;
        let pixel_y = window.innerHeight * y;
        element.style.top = `${pixel_y}px`;
        element.style.left = `${pixel_x}px`;
      }
    };

    let spawn_cursor_cb = function (id, icon) {
      console.log(`spawning ${id}`);
      let element = document.createElement("div");
      element.className = "server-cursor";
      id_map[id] = element;
      try {
        element.style.top = `0px`;
        element.style.left = `0px`;
        element.style.position = `fixed`;
        element.style.background = `red`;
        element.style.width = `10px`;
        element.style.height = `10px`;
        element.style.zIndex = `99999`;

        document.getElementsByTagName("html")[0].append(element);
      } catch (e) {
        console.log(e);
      }
    };

    let despawn_cursor_cb = function (id) {
      if (id in id_map) {
        console.log(`despawning ${id}`);
        let element = id_map[id];
        element.remove();
        delete id_map[id];
      }
    };

    let r = start_websocket(
      move_cursor_cb,
      spawn_cursor_cb,
      despawn_cursor_cb,
      window.location.hostname
    );

    console.log(r);

    let hmtl = document.getElementsByTagName("html")[0];

    hmtl.onmousemove = (ev) => {
      let percent_x = ev.pageX / window.innerWidth;
      let percent_y = ev.pageY / window.innerHeight;
      send_location(r, percent_x, percent_y);
    };
  });
})();
