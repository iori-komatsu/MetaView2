import React, { useEffect, useRef, useState } from "react";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import "./App.css";

function ImageCard({ image, focusOn }) {
  const el = useRef(null);
  useEffect(() => {
    if (focusOn) {
      el.current.scrollIntoView();
    }
  });

  const handleCloseButtonClick = () => {
    invoke("remove_image", { id: image.id }).catch(console.error);
  }

  const metadataItems = image.metadata.map((kv, index) =>
    <React.Fragment key={index}>
      <dt>{kv[0]}</dt>
      <dd>{kv[1]}</dd>
    </React.Fragment>
  );

  return (
    <div className="card mb-3" ref={el}>
      <div className="row g-0">
        <div className="col-md-4">
          <img className="card-img-top h-100" src={convertFileSrc(image.filename)} style={{objectFit: "contain"}} />
        </div>
        <div className="col-md-8 position-relative">
          <button type="button" className="btn-close position-absolute top-0 end-0" aria-label="Close" onClick={handleCloseButtonClick}></button>
          <div className="card-body">
            {metadataItems}
            <p className="card-text"><small className="text-muted">{image.filename}</small></p>
          </div>
        </div>
      </div>
    </div>
  );
}

function App() {
  const [images, setImages] = useState([]);
  const [focusOn, setFocusOn] = useState(0);

  useEffect(() => {
    console.log("Listen file-drop");
    const unlisten = listen("tauri://file-drop", event => {
      if (event.payload.length == 0) {
        return;
      }
      const filename = event.payload[0];
      invoke("add_image", { filename }).catch(alert);
    });

    return () => {
      unlisten.then(f => {
        console.log("Unlisten file-drop");
        f();
      }).catch(console.error);
    };
  }, []);

  useEffect(() => {
    console.log("Listen state-changed");
    const unlisten = listen("state-changed", event => {
      setImages(event.payload.images);
      setFocusOn(event.payload.focus_on);
    });

    return () => {
      unlisten.then(f => {
        console.log("Unlisten stage-changed");
        f();
      }).catch(console.error);
    };
  }, []);

  if (images.length == 0) {
    return (
      <div className="container-fluid">
        <div className="row" style={{padding: "10px"}}>
          <div className="alert alert-primary" role="alert">
            Drag and drop PNG files here!
          </div>
        </div>
      </div>
    );
  }

  const imageCards = images.map(image => {
    return (
      <React.Fragment key={image.id}>
        <ImageCard image={image} focusOn={image.id == focusOn} />
      </React.Fragment>
    );
  })

  return (
    <div className="container-fluid" style={{padding: "8px"}}>
      {imageCards}
    </div>
  );
}

export default App;
