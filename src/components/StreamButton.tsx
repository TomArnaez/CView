import React, { useState} from 'react';
import { Button } from '@mantine/core';
import classes from "../css/StreamButton.module.css";

const StreamButton = () => {
    const [isLive, setIsLive] = useState<boolean>(false);
    const [isDisabled, setDisabled] = useState<boolean>(false);
}