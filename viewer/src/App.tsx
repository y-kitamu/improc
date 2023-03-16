import React, { ReactEventHandler, Suspense, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Canvas, useLoader } from "@react-three/fiber";
import { dialog } from "@tauri-apps/api";
import { Button } from "@mui/material";
import * as THREE from "three";

type Image = {
  size: number[];
  data: number[];
};

type ImageProps = {
  selected: string[];
};

const Image = (props: ImageProps) => {
  const { selected } = props;
  const [map, setMap] = useState<THREE.texture>();

  useEffect(() => {
    const loadTexture = async () => {
      if (selected.length === 0) {
        return;
      }
      const allImages: Image[] = await Promise.all(
        selected.map(async (path: string) => {
          return await invoke("read_image", { path: path });
        })
      );

      const image = allImages[0];
      const texture = new THREE.DataArrayTexture(
        image.data,
        image.size[0],
        image.size[1],
        1
      );
      setMap(texture);
    };

    const texture = useLoader(THREE.TextureLoader, selected[0]);
    setMap(texture);
  }, [selected]);

  console.log("map : ", map);
  // <ambientLight intensity={0.1} />
  // <directionalLight color="red" position={[0, 0, 5]} />
  // <sprite position={[1, 1, 0]}>
  //   {{ map } && <spriteMaterial map={map} />}
  // </sprite>
  // <mesh position={[0, 0, 0]}>
  //   <sphereGeometry args={[3, 3, 3]} />
  //   <meshBasicMaterial color="blue" map={map} />
  // </mesh>

  return (
    <>
      <Suspense fallback={null}>
        {map && <primitive object={map.scene} />}
      </Suspense>
    </>
  );
};

const App = () => {
  const [greetMsg, setGreetMsg] = useState("");
  const [filelist, setFilelist] = useState<string[]>([]);

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
      } else if (Array.isArray(selected)) {
        setFilelist(selected);
      }
    })();
  };

  return (
    <div className="container">
      <button onClick={onClickButton}>Open</button>
      <Button variant="contained" onClick={onClickButton}>
        Open by mui
      </Button>
      <Canvas>
        <Image selected={filelist} />
      </Canvas>
    </div>
  );
};

export default App;
