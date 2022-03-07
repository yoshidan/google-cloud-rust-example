#!/usr/bin/env python

# Copyright 2015 Google Inc. All rights reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import time
from locust import HttpUser, SequentialTaskSet, task, between, constant

class UserBehavior(SequentialTaskSet):
    @task(1)
    def read_inventory(self):
        t1 = time.time()
        response = self.client.post(url='/Publish', data={'message': str(t1) })

class WebsiteUser(HttpUser):
    tasks = {UserBehavior:1}
    wait_time = constant(0.2)