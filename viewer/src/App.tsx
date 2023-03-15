import React, { ReactEventHandler, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Canvas } from "@react-three/fiber";
import { dialog } from "@tauri-apps/api";
import { Button } from "@mui/material";
import * as THREE from "three";

type Image = {
  size: number[];
  data: number[];
};

const App = () => {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const greet = async () => {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name }));
  };

  const onClickButton = (
    event: React.MouseEvent<HTMLButtonElement, MouseEvent>
  ) => {
    (async () => {
      const selected = await dialog.open({
        multiple: true,
        filters: [
          {
            name: "Image",
            extensions: ["jpg", "png", "gif"],
          },
        ],
      });

      if (selected === null) {
        console.log("no file selected");
        return;
      }
      const allImages: Image[] = Promise.all(
        selected.map(async (path) => {
          return await invoke("read_image", { path: path });
        })
      );

      const image = allImages[0];
      const texture = THREE.DataArrayTexture(
        image.data,
        image.size[0],
        image.size[1],
        1
      );
    })();
  };

  return (
    <div className="container">
      <button onClick={onClickButton}>Open</button>
      <Button variant="contained" onClick={onClickButton}>
        Open by mui
      </Button>
      <Canvas>
        <ambientLight intensity={0.1} />
        <directionalLight color="red" position={[0, 0, 5]} />
        <mesh position={[2, 2, 0]}>
          <boxGeometry args={[2, 2, 2]} />
          <shaderMaterial />
        </mesh>
        <mesh position={[-2, -2, 0]}>
          <sphereGeometry args={[1, 16, 16]} />
          <meshBasicMaterial color="blue" />
        </mesh>
      </Canvas>
    </div>
  );
};

export default App;
